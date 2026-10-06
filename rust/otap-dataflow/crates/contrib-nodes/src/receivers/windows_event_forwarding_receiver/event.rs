// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Convert forwarded Windows event XML into an atomic OTAP log batch.
//!
//! # Severity and event codes
//!
//! Windows `System/Level` controls `severity_number` using the base number of
//! each OpenTelemetry severity range:
//!
//! | Windows level | Windows meaning | OpenTelemetry severity | Number |
//! | --- | --- | --- | --- |
//! | 1 | Critical | FATAL | 21 |
//! | 2 | Error | ERROR | 17 |
//! | 3 | Warning | WARN | 13 |
//! | 4 | Informational | INFO | 9 |
//! | 5 | Verbose | DEBUG | 5 |
//! | 0 or absent | LogAlways or unspecified | UNSPECIFIED | 0 |
//! | Other valid `u8` values | Unmapped provider level | UNSPECIFIED | 0 |
//!
//! A nonnumeric level or a value outside `0..=255` rejects the batch; it does not
//! fall back to UNSPECIFIED. `RenderingInfo/Level` independently supplies the
//! optional, potentially localized `severity_text`. It is neither interpreted
//! to infer severity nor synthesized when absent. The raw numeric Windows level
//! is not currently emitted as a separate attribute.
//!
//! `System/EventID` is a provider-defined event identifier, not a severity or a
//! Win32 error code. It must parse as `u32`; its original text is retained in
//! `winlog.event_id`, while its parsed decimal value forms `event_name` as
//! `<channel>:<event_id>`. A missing channel produces an empty channel prefix.
//! Provider payload fields such as `ErrorCode`, HRESULT, or NTSTATUS values are
//! not decoded or translated into severity. They remain payload values under
//! the normal EventData or structured-body mapping.
//!
//! # Log mapping
//!
//! - `System/TimeCreated/@SystemTime` supplies the event timestamp. One collector
//!   observation timestamp is shared by all records in the input batch.
//! - A supplied `RenderingInfo/Message`, including an empty message, becomes the
//!   string body. Only an absent message selects a structured map body containing
//!   available EventData and UserData; absent payload sections yield an empty map.
//! - EventData values also become attributes named `winlog.event_data.*`. Unique
//!   names yield strings; repeated names (including generated `paramN` collisions)
//!   yield arrays of strings in source order. Each attribute key appears once.
//!   Available system metadata becomes `winlog.*` string attributes.
//! - Authenticated principal, peer address, and subscription identity become
//!   `source.*` and `wef.subscription.*` attributes. The delivery version is
//!   retained verbatim rather than interpreted or compared here.
//! - Original XML is included as `event.original` only when requested.
//!
//! Synthetic subscription bookmark events are validated and omitted from logs.
//! This module does not acknowledge deliveries or advance bookmarks; its caller
//! handles progress only after conversion and downstream acceptance.

use super::xml::{Document, Node};
use otel_arrow_dfe_pdata::{
    encode::record::{
        attributes::StrKeysAttributesRecordBatchBuilder, logs::LogsRecordBatchBuilder,
    },
    otap::{Logs, OtapArrowRecords},
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
};
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

const EVENT_NS: &str = "http://schemas.microsoft.com/win/2004/08/events/event";

/// Convert a complete batch without returning partially encoded records.
///
/// Accepts 1 through `u16::MAX` input events. Returns `Ok(None)` when every event
/// is a validated synthetic bookmark marker, or `Ok(Some(_))` for the ordinary
/// events remaining after marker filtering. See the module documentation for
/// severity, event-code, body, and attribute mappings.
///
/// # Errors
///
/// Returns a diagnostic string, not a numeric error code. Any conversion error
/// rejects the entire batch, including ordinary events encoded earlier in the
/// loop. Failure conditions include:
///
/// - Empty or oversized input batches, malformed XML, or more than 100,000 XML
///   nodes in an individual event.
/// - An unexpected root namespace/name, missing required System fields, duplicate
///   fields encountered during extraction, or nested elements in scalar fields.
/// - Invalid EventID or Level numbers, invalid RFC 3339 timestamps, or timestamps
///   outside the nonnegative signed 64-bit nanosecond range.
/// - Malformed recognized bookmark markers or fallback UserData nesting beyond
///   depth 32 (the UserData wrapper is depth zero).
/// - Clock conversion, structured-body serialization, or Arrow construction errors.
///
/// These diagnostics describe receiver conversion failures, not errors reported
/// by the Windows event itself. Transport response selection belongs to the caller.
pub fn encode_events(
    events: &[String],
    principal: &str,
    peer: &str,
    subscription_name: &str,
    subscription_id: uuid::Uuid,
    version: &str,
    include_event_original: bool,
) -> Result<Option<OtapArrowRecords>, String> {
    if events.is_empty() || events.len() > usize::from(u16::MAX) {
        return Err("invalid event batch size".into());
    }
    let observed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let observed = i64::try_from(observed).map_err(|error| error.to_string())?;
    let mut logs = LogsRecordBatchBuilder::new();
    let mut attributes = StrKeysAttributesRecordBatchBuilder::<u16>::new();
    let mut count: usize = 0;
    for xml in events {
        let document = Document::parse_bounded(xml, 100_000).map_err(|error| error.to_string())?;
        let root = document.root_element();
        if !root.has_tag_name((EVENT_NS, "Event")) {
            return Err("expected Windows Event root".into());
        }
        let system = child(root, "System")?.ok_or("missing System")?;
        let event_id = field(system, "EventID")?.ok_or("missing EventID")?;
        let event_id_number = event_id.parse::<u32>().map_err(|_| "invalid EventID")?;
        let time = child(system, "TimeCreated")?
            .and_then(|node| node.attribute("SystemTime"))
            .ok_or("missing SystemTime")?;
        let timestamp = chrono::DateTime::parse_from_rfc3339(time)
            .map_err(|_| "invalid SystemTime")?
            .timestamp_nanos_opt()
            .filter(|value| *value >= 0)
            .ok_or("SystemTime outside supported range")?;
        let level = field(system, "Level")?
            .map(|value| value.parse::<u8>().map_err(|_| "invalid Level"))
            .transpose()?
            .unwrap_or_default();
        let rendering = child(root, "RenderingInfo")?;
        let message = rendering
            .map(|node| field(node, "Message"))
            .transpose()?
            .flatten();
        let severity_text = rendering
            .map(|node| field(node, "Level"))
            .transpose()?
            .flatten();
        let channel = field(system, "Channel")?.unwrap_or_default();
        if is_bookmark_event(root, system, event_id_number, &channel)? {
            continue;
        }
        let record_id = u16::try_from(count).map_err(|error| error.to_string())?;
        logs.append_id(Some(record_id));
        logs.append_time_unix_nano(timestamp);
        logs.append_observed_time_unix_nano(observed);
        logs.append_severity_number(Some(match level {
            1 => 21,
            2 => 17,
            3 => 13,
            4 => 9,
            5 => 5,
            _ => 0,
        }));
        logs.append_severity_text(severity_text.as_deref().map(str::as_bytes));
        logs.append_event_name(Some(format!("{channel}:{event_id_number}").as_bytes()));
        if let Some(message) = message {
            logs.body.append_str(message.as_bytes());
        } else {
            let body = structured_body(root)?;
            logs.body.append_map(&body);
        }
        let mut attr = |key: &str, value: &str| {
            attributes.append_key(key);
            attributes.any_values_builder.append_str(value.as_bytes());
            attributes.append_parent_id(&record_id);
        };
        for (key, value) in [
            ("source.principal", principal),
            ("source.address", peer),
            ("wef.subscription.name", subscription_name),
            ("wef.subscription.uuid", &subscription_id.to_string()),
            ("wef.subscription.version", version),
            ("winlog.event_id", event_id.as_str()),
            ("winlog.channel", channel.as_str()),
        ] {
            attr(key, value);
        }
        if include_event_original {
            attr("event.original", xml);
        }
        for (element, key) in [
            ("EventRecordID", "winlog.record_id"),
            ("Task", "winlog.task"),
            ("Opcode", "winlog.opcode"),
            ("Keywords", "winlog.keywords"),
            ("Computer", "winlog.computer"),
        ] {
            if let Some(value) = field(system, element)? {
                attr(key, &value);
            }
        }
        if let Some(provider) = child(system, "Provider")? {
            for (name, key) in [
                ("Name", "winlog.provider.name"),
                ("Guid", "winlog.provider.guid"),
            ] {
                if let Some(value) = provider.attribute(name) {
                    attr(key, value);
                }
            }
        }
        if let Some(security) = child(system, "Security")?
            && let Some(sid) = security.attribute("UserID")
        {
            attr("winlog.user.identifier", sid);
        }
        if let Some(data) = child(root, "EventData")? {
            let mut values_by_name: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for (position, item) in data.children().filter(Node::is_element).enumerate() {
                if !item.has_tag_name((EVENT_NS, "Data")) {
                    continue;
                }
                let name = item
                    .attribute("Name")
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("param{}", position + 1));
                values_by_name.entry(name).or_default().push(text(item)?);
            }
            for (name, values) in values_by_name {
                attributes.append_key(&format!("winlog.event_data.{name}"));
                attributes.append_parent_id(&record_id);
                if values.len() == 1 {
                    attributes
                        .any_values_builder
                        .append_str(values[0].as_bytes());
                } else {
                    let encoded = serde_cbor::to_vec(&values).map_err(|error| error.to_string())?;
                    attributes.any_values_builder.append_slice(&encoded);
                }
            }
        }
        count += 1;
    }
    if count == 0 {
        return Ok(None);
    }
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
        .append_name_n(Some(b"windows_event_forwarding"), count);
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

/// Encode a fallback map as CBOR for the OTAP builder's native map representation.
///
/// EventData is an ordered list of name/value entries, retaining repeated names.
/// Unnamed entries use their one-based element position as `paramN`. UserData
/// retains its namespace-aware element tree rather than flattening provider keys.
fn structured_body(root: Node<'_, '_>) -> Result<Vec<u8>, String> {
    let mut body = serde_json::Map::new();
    if let Some(data) = child(root, "EventData")? {
        let mut entries = Vec::new();
        for (position, item) in data.children().filter(Node::is_element).enumerate() {
            if item.has_tag_name((EVENT_NS, "Data")) {
                entries.push(serde_json::json!({
                    "name": item.attribute("Name").map(str::to_owned).unwrap_or_else(|| format!("param{}", position + 1)),
                    "value": text(item)?,
                }));
            }
        }
        let _ = body.insert("EventData".into(), entries.into());
    }
    if let Some(data) = child(root, "UserData")? {
        let _ = body.insert("UserData".into(), structured_element(data, 0)?);
    }
    serde_cbor::to_vec(&body).map_err(|error| error.to_string())
}

/// Preserve element/attribute names, namespaces, and ordered mixed text/children.
///
/// The recursion limit applies to the fallback body only; rendered-message events
/// do not traverse UserData through this helper. Comments and processing
/// instructions are omitted from the structured representation.
fn structured_element(node: Node<'_, '_>, depth: usize) -> Result<serde_json::Value, String> {
    if depth > 32 {
        return Err("UserData exceeds maximum nesting depth of 32".into());
    }
    let attributes: Vec<_> = node
        .attributes()
        .map(|attribute| {
            serde_json::json!({
                "name": attribute.name(),
                "namespace": attribute.namespace().unwrap_or_default(),
                "value": attribute.value(),
            })
        })
        .collect();
    let mut content = Vec::new();
    for child in node.children() {
        if child.is_element() {
            content.push(structured_element(child, depth + 1)?);
        } else if let Some(text) = child.text().filter(|_| child.is_text()) {
            content.push(serde_json::Value::String(text.to_owned()));
        }
    }
    Ok(serde_json::json!({
        "name": node.tag_name().name(),
        "namespace": node.tag_name().namespace().unwrap_or_default(),
        "attributes": attributes,
        "content": content,
    }))
}

/// Recognize a synthetic marker by provider, event ID 111, empty channel, and shape.
///
/// Event ID 111 alone never suppresses an ordinary event. A recognized candidate
/// must contain a scalar SubscriptionId (which may be empty) and no unexpected
/// marker children. This validates marker structure, not subscription ownership.
fn is_bookmark_event(
    root: Node<'_, '_>,
    system: Node<'_, '_>,
    event_id: u32,
    channel: &str,
) -> Result<bool, String> {
    if event_id != 111
        || !channel.is_empty()
        || child(system, "Provider")?.and_then(|provider| provider.attribute("Name"))
            != Some("Microsoft-Windows-EventForwarder")
    {
        return Ok(false);
    }
    let Some(marker) = child(root, "SubscriptionBookmarkEvent")? else {
        return Ok(false);
    };
    if root.children().filter(Node::is_element).any(|node| {
        !node.has_tag_name((EVENT_NS, "System"))
            && !node.has_tag_name((EVENT_NS, "SubscriptionBookmarkEvent"))
    }) {
        return Ok(false);
    }
    if marker
        .children()
        .filter(Node::is_element)
        .any(|node| !node.has_tag_name((EVENT_NS, "SubscriptionId")))
        || marker
            .children()
            .filter(Node::is_text)
            .any(|node| node.text().is_some_and(|text| !text.trim().is_empty()))
        || field(marker, "SubscriptionId")?.is_none()
    {
        return Err("invalid subscription bookmark event".into());
    }
    Ok(true)
}

/// Find a unique direct child in the Windows event namespace, rejecting duplicates.
fn child<'node, 'input>(
    node: Node<'node, 'input>,
    name: &str,
) -> Result<Option<Node<'node, 'input>>, String> {
    let mut matches = node
        .children()
        .filter(|child| child.has_tag_name((EVENT_NS, name)));
    let first = matches.next();
    if matches.next().is_some() {
        return Err(format!("duplicate {name}"));
    }
    Ok(first)
}

/// Join decoded scalar text without trimming whitespace; reject nested elements.
fn text(node: Node<'_, '_>) -> Result<String, String> {
    if node.first_element_child().is_some() {
        return Err("unexpected nested event field".into());
    }
    Ok(node
        .children()
        .filter(Node::is_text)
        .filter_map(|child| child.text())
        .collect())
}

/// Read an optional unique scalar field, distinguishing absence from empty text.
fn field(node: Node<'_, '_>, name: &str) -> Result<Option<String>, String> {
    child(node, name)?.map(text).transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: rendered and fallback events contain repeated names and named/unnamed paramN collisions.
    /// Guarantees: attribute keys are unique per record and collisions retain every value in source order.
    #[test]
    fn preserves_colliding_event_data_as_arrays() {
        use arrow::{
            array::{Array, BinaryArray, StringArray, UInt16Array},
            datatypes::DataType,
        };
        let payload = "<EventData><Data Name='repeat'>first</Data><Data Name='repeat'/><Data>unnamed</Data><Data Name='param3'>named</Data><Data Name='unique'>single</Data></EventData>";
        let events: Vec<_> = ["", "<RenderingInfo><Message>rendered</Message></RenderingInfo>"]
            .into_iter()
            .map(|rendering| format!("<Event xmlns='{EVENT_NS}'><System><EventID>42</EventID><TimeCreated SystemTime='2026-09-22T19:28:11Z'/></System>{payload}{rendering}</Event>"))
            .collect();
        let records = encode_events(
            &events,
            "host",
            "127.0.0.1",
            "test",
            uuid::Uuid::nil(),
            "opaque",
            false,
        )
        .unwrap()
        .unwrap();
        let attributes = records.get(ArrowPayloadType::LogAttrs).unwrap();
        let keys = arrow::compute::cast(attributes.column_by_name("key").unwrap(), &DataType::Utf8)
            .unwrap();
        let keys = keys.as_any().downcast_ref::<StringArray>().unwrap();
        let strings =
            arrow::compute::cast(attributes.column_by_name("str").unwrap(), &DataType::Utf8)
                .unwrap();
        let strings = strings.as_any().downcast_ref::<StringArray>().unwrap();
        let serialized =
            arrow::compute::cast(attributes.column_by_name("ser").unwrap(), &DataType::Binary)
                .unwrap();
        let serialized = serialized.as_any().downcast_ref::<BinaryArray>().unwrap();
        let parents = attributes
            .column_by_name("parent_id")
            .unwrap()
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap();
        let mut seen = std::collections::HashSet::new();
        let mut matched = 0;
        for index in 0..attributes.num_rows() {
            let key = keys.value(index);
            assert!(seen.insert((parents.value(index), key)));
            match key {
                "winlog.event_data.repeat" | "winlog.event_data.param3" => {
                    assert!(strings.is_null(index));
                    let values: Vec<String> =
                        serde_cbor::from_slice(serialized.value(index)).unwrap();
                    assert_eq!(
                        values,
                        if key.ends_with("repeat") {
                            vec!["first", ""]
                        } else {
                            vec!["unnamed", "named"]
                        }
                    );
                    matched += 1;
                }
                "winlog.event_data.unique" => {
                    assert_eq!(strings.value(index), "single");
                    assert!(serialized.is_null(index));
                    matched += 1;
                }
                _ => {}
            }
        }
        assert_eq!(matched, 6);
    }

    /// Scenario: messages are absent, rendered, empty, or carry repeated EventData and namespaced UserData.
    /// Guarantees: fallback bodies are native maps, preserve data order and namespaces, and never replace a supplied message.
    #[test]
    fn encodes_structured_fallback_bodies() {
        use arrow::array::{Array, BinaryArray, StringArray, StructArray};
        use arrow::datatypes::DataType;

        let wrap = |payload: &str| {
            format!(
                r#"<Event xmlns="{EVENT_NS}"><System><Provider Name="Example"/><EventID>42</EventID><TimeCreated SystemTime="2026-09-22T19:28:11Z"/></System>{payload}</Event>"#
            )
        };
        let data = r#"<EventData><Data Name="repeat">first</Data><Data Name="repeat">second</Data><Data>{"ok":true}</Data><Data Name="empty"/></EventData>"#;
        let user_data = r#"<UserData><p:Payload xmlns:p="urn:provider" xmlns:q="urn:attribute" q:code="7">before<p:Value>one &amp; two</p:Value>between<p:Value/>after</p:Payload></UserData>"#;
        let events = vec![
            wrap(data),
            wrap(&format!(
                "{data}<RenderingInfo><Message>rendered &amp; decoded</Message></RenderingInfo>"
            )),
            wrap(user_data),
            wrap(""),
            wrap("<RenderingInfo><Message/></RenderingInfo>"),
            wrap(&format!("{data}{user_data}")),
        ];
        for include_original in [false, true] {
            let records = encode_events(
                &events,
                "host",
                "127.0.0.1",
                "test",
                uuid::Uuid::nil(),
                "opaque",
                include_original,
            )
            .unwrap()
            .unwrap();
            let logs = records.get(ArrowPayloadType::Logs).unwrap();
            let body = logs
                .column_by_name("body")
                .unwrap()
                .as_any()
                .downcast_ref::<StructArray>()
                .unwrap();
            let serialized =
                arrow::compute::cast(body.column_by_name("ser").unwrap(), &DataType::Binary)
                    .unwrap();
            let serialized = serialized.as_any().downcast_ref::<BinaryArray>().unwrap();
            let strings =
                arrow::compute::cast(body.column_by_name("str").unwrap(), &DataType::Utf8).unwrap();
            let strings = strings.as_any().downcast_ref::<StringArray>().unwrap();
            let decode = |index| {
                serde_cbor::from_slice::<serde_json::Value>(serialized.value(index)).unwrap()
            };
            let expected = serde_json::json!({"EventData": [
                {"name": "repeat", "value": "first"},
                {"name": "repeat", "value": "second"},
                {"name": "param3", "value": "{\"ok\":true}"},
                {"name": "empty", "value": ""},
            ]});
            assert_eq!(decode(0), expected);
            assert!(strings.is_null(0));
            assert!(serialized.is_null(1));
            assert_eq!(strings.value(1), "rendered & decoded");
            assert_eq!(decode(3), serde_json::json!({}));
            assert!(serialized.is_null(4));
            assert_eq!(strings.value(4), "");
            let user = decode(2);
            let payload = &user["UserData"]["content"][0];
            assert_eq!(payload["name"], "Payload");
            assert_eq!(payload["namespace"], "urn:provider");
            assert_eq!(
                payload["attributes"],
                serde_json::json!([{"name": "code", "namespace": "urn:attribute", "value": "7"}])
            );
            assert_eq!(payload["content"][0], "before");
            assert_eq!(payload["content"][1]["content"][0], "one & two");
            assert_eq!(payload["content"][2], "between");
            assert_eq!(payload["content"][3]["name"], "Value");
            assert_eq!(payload["content"][3]["content"], serde_json::json!([]));
            assert_eq!(payload["content"][4], "after");
            assert_eq!(decode(5)["EventData"], expected["EventData"]);
            assert_eq!(decode(5)["UserData"], user["UserData"]);
        }
    }

    /// Scenario: fallback UserData reaches the nesting bound or duplicate data sections arrive in a batch.
    /// Guarantees: bounded trees encode successfully and invalid fallback data rejects the entire batch.
    #[test]
    fn bounds_structured_fallback() {
        let wrap = |payload: &str| {
            format!(
                r#"<Event xmlns="{EVENT_NS}"><System><EventID>42</EventID><TimeCreated SystemTime="2026-09-22T19:28:11Z"/></System>{payload}</Event>"#
            )
        };
        let encode = |events: &[String]| {
            encode_events(
                events,
                "host",
                "127.0.0.1",
                "test",
                uuid::Uuid::nil(),
                "opaque",
                false,
            )
        };
        for depth in [32, 33] {
            let nested = format!(
                "<UserData>{}{}</UserData>",
                "<Nested>".repeat(depth),
                "</Nested>".repeat(depth)
            );
            assert_eq!(encode(&[wrap(&nested)]).is_ok(), depth == 32);
        }
        for invalid in [
            "<EventData/><EventData/>",
            "<UserData/><UserData/>",
            "<EventData><Data><Nested/></Data></EventData>",
        ] {
            assert!(encode(&[wrap(""), wrap(invalid)]).is_err());
        }
    }

    /// Scenario: a rendered Windows event contains system metadata, named data, and escaped message text.
    /// Guarantees: Arrow conversion succeeds as one batch and rejects malformed timestamps or ambiguous system fields.
    #[test]
    fn maps_rendered_events() {
        let xml = format!(
            r#"<Event xmlns="{EVENT_NS}"><System><Provider Name="Example"/><EventID>42</EventID><Level>2</Level><TimeCreated SystemTime="2026-09-22T19:28:11.1234567Z"/><Channel>Application</Channel></System><EventData><Data Name="name">value</Data></EventData><RenderingInfo><Message>A &amp; B</Message><Level>Error</Level></RenderingInfo></Event>"#
        );
        let encode = |events: Vec<String>| {
            encode_events(
                &events,
                "host",
                "127.0.0.1",
                "test",
                uuid::Uuid::nil(),
                "old%2Fversion",
                false,
            )
            .map(|records| records.expect("ordinary event batch"))
        };
        let records = encode(vec![xml.clone(), xml.clone()]).unwrap();
        let logs = records.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(logs.num_rows(), 2);
        let severity = arrow::compute::cast(
            logs.column_by_name("severity_number").unwrap(),
            &arrow::datatypes::DataType::Int32,
        )
        .unwrap();
        assert_eq!(
            severity
                .as_any()
                .downcast_ref::<arrow::array::Int32Array>()
                .unwrap()
                .values()
                .as_ref(),
            &[17, 17]
        );
        let attrs = records.get(ArrowPayloadType::LogAttrs).unwrap();
        let keys = arrow::compute::cast(
            attrs.column_by_name("key").unwrap(),
            &arrow::datatypes::DataType::Utf8,
        )
        .unwrap();
        let values = arrow::compute::cast(
            attrs.column_by_name("str").unwrap(),
            &arrow::datatypes::DataType::Utf8,
        )
        .unwrap();
        let keys = keys
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .unwrap();
        let values = values
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .unwrap();
        let pairs: Vec<_> = keys.iter().zip(values.iter()).collect();
        assert_eq!(
            pairs
                .iter()
                .filter(|pair| **pair == (Some("wef.subscription.version"), Some("old%2Fversion")))
                .count(),
            2
        );
        assert!(pairs.contains(&(Some("source.principal"), Some("host"))));
        assert!(pairs.contains(&(Some("winlog.event_data.name"), Some("value"))));
        assert!(!pairs.iter().any(|(key, _)| *key == Some("event.original")));
        let with_original = encode_events(
            &[xml.clone(), xml.clone()],
            "host",
            "127.0.0.1",
            "test",
            uuid::Uuid::nil(),
            "old%2Fversion",
            true,
        )
        .unwrap()
        .unwrap();
        let original_logs = with_original.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(logs.schema(), original_logs.schema());
        for field in logs.schema().fields() {
            if field.name() != "observed_time_unix_nano" {
                assert_eq!(
                    logs.column_by_name(field.name()),
                    original_logs.column_by_name(field.name())
                );
            }
        }
        let original_attrs = with_original.get(ArrowPayloadType::LogAttrs).unwrap();
        let original_keys = arrow::compute::cast(
            original_attrs.column_by_name("key").unwrap(),
            &arrow::datatypes::DataType::Utf8,
        )
        .unwrap();
        let original_values = arrow::compute::cast(
            original_attrs.column_by_name("str").unwrap(),
            &arrow::datatypes::DataType::Utf8,
        )
        .unwrap();
        let original_keys = original_keys
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .unwrap();
        let original_values = original_values
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .unwrap();
        assert_eq!(
            original_keys
                .iter()
                .zip(original_values.iter())
                .filter(
                    |(key, value)| *key == Some("event.original") && *value == Some(xml.as_str())
                )
                .count(),
            2
        );
        assert!(encode(vec![xml.replace("2026-09-22T19:28:11.1234567Z", "invalid")]).is_err());
        assert!(
            encode(vec![xml.replace(
                "<EventID>42</EventID>",
                "<EventID>42</EventID><EventID>43</EventID>"
            )])
            .is_err()
        );
        assert!(encode(vec![xml, "<bad/>".into()]).is_err());
        assert!(encode(Vec::new()).is_err());
    }

    /// Scenario: synthetic bookmark markers arrive alone, mixed with ordinary events, or followed by invalid XML.
    /// Guarantees: only the known control shape is suppressed, emitted IDs are contiguous, and malformed batches fail atomically.
    #[test]
    fn separates_bookmark_events_from_logs() {
        let marker = format!(
            r#"<Event xmlns="{EVENT_NS}"><System><Provider Name="Microsoft-Windows-EventForwarder"/><EventID>111</EventID><TimeCreated SystemTime="2026-09-22T19:44:13.247Z"/><Computer>host</Computer></System><SubscriptionBookmarkEvent><SubscriptionId/></SubscriptionBookmarkEvent></Event>"#
        );
        let ordinary = marker.replace("Microsoft-Windows-EventForwarder", "ApplicationProvider");
        let encode = |events: Vec<String>| {
            encode_events(
                &events,
                "host",
                "127.0.0.1",
                "test",
                uuid::Uuid::nil(),
                "old-version",
                false,
            )
        };
        assert!(encode(vec![marker.clone()]).unwrap().is_none());
        let records = encode(vec![
            marker.clone(),
            ordinary.clone(),
            marker.clone(),
            ordinary.clone(),
        ])
        .unwrap()
        .unwrap();
        let logs = records.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(logs.num_rows(), 2);
        let ids = logs
            .column_by_name("id")
            .unwrap()
            .as_any()
            .downcast_ref::<arrow::array::UInt16Array>()
            .unwrap();
        assert_eq!(ids.values().as_ref(), &[0, 1]);
        for non_marker in [
            marker.replace("<EventID>111</EventID>", "<EventID>112</EventID>"),
            marker.replace(
                "<SubscriptionBookmarkEvent>",
                "<SubscriptionBookmarkEvent xmlns=\"urn:other\">",
            ),
            marker.replace("</System>", "<Channel>Application</Channel></System>"),
        ] {
            assert!(encode(vec![non_marker]).unwrap().is_some());
        }
        assert!(encode(vec![marker.clone(), "<bad/>".into()]).is_err());
        assert!(encode(vec![marker.clone(), ordinary, "<bad/>".into()]).is_err());
        assert!(
            encode(vec![marker.replace(
                "<SubscriptionId/>",
                "<SubscriptionId/><SubscriptionId/>"
            )])
            .is_err()
        );
    }
}
