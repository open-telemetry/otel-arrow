// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Atomic conversion of rendered WEF events through the shared Windows mapper.
//!
//! Ordinary records retain shared `windows.eventlog.*` semantics. Transport
//! identity is supplied separately from event claims. Recognized synthetic
//! bookmark events are validated and omitted from logs, but no bookmark is
//! extracted or committed here. Successful conversion is not downstream
//! acceptance and must never alone trigger a protocol acknowledgement.

use super::{config::Limits, identity::SourceIdentity};
use crate::common::{
    windows_event::{
        map::{self, AttrValue},
        model::{Content, Payload, WindowsEvent},
        parse,
    },
    xml::Document,
};
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use uuid::Uuid;

// The shared OTAP log mapper uses u16 record identifiers.
pub(super) const MAX_EVENTS: usize = u16::MAX as usize;
// Bound each inner event's parser bookkeeping independently of its byte limit.
const MAX_EVENT_NODES: usize = 100_000;
const EVENT_NS: &str = "http://schemas.microsoft.com/win/2004/08/events/event";

/// Trusted transport and configured subscription metadata for a delivery.
pub struct MappingContext<'a> {
    /// Authorized TLS identity, never taken from SOAP or Event/System/Computer.
    pub source: &'a SourceIdentity,
    /// Transport peer address.
    pub peer: &'a str,
    /// Configured subscription name.
    pub subscription_name: &'a str,
    /// Stable configured subscription identifier.
    pub subscription_id: Uuid,
    /// Opaque version from the delivery route, not compared to the current version.
    pub version: &'a str,
}

/// Convert an entire extracted batch without publishing partial output.
///
/// The batch must contain 1 through 65,535 event documents. Each document is
/// bounded by `max_event_bytes` and 100,000 XML nodes; their combined UTF-8 size
/// is bounded by `max_decompressed_bytes`, including synthetic markers. Original
/// XML is retained only when requested, after SOAP text decoding and without
/// inner-XML reserialization. All records share the caller's observation time.
///
/// Returns `Ok(None)` only for a nonempty batch of validated synthetic markers.
/// This does not authorize local progress; future runtime code must apply
/// ordering, capacity, bookmark validation, and downstream acceptance rules.
///
/// # Errors
///
/// Rejects empty/oversized batches, malformed XML or Windows fields, malformed
/// recognized markers, negative observation times, and mapping failures. Any
/// failure rejects the complete batch, including previously decoded events.
pub fn encode_events(
    events: &[String],
    context: &MappingContext<'_>,
    limits: &Limits,
    include_event_original: bool,
    observed_time_unix_nano: i64,
) -> Result<Option<OtapArrowRecords>, String> {
    if events.is_empty() || events.len() > MAX_EVENTS {
        return Err("invalid event batch size".into());
    }
    if observed_time_unix_nano < 0 {
        return Err("negative observation time".into());
    }
    let mut total_bytes = 0usize;
    for xml in events {
        if xml.len() > limits.max_event_bytes
            || xml.len() > limits.max_decompressed_bytes.saturating_sub(total_bytes)
        {
            return Err("event batch exceeds byte limits".into());
        }
        total_bytes += xml.len();
    }
    let mut decoded = Vec::new();
    let mut originals = Vec::new();
    for xml in events {
        let document =
            Document::parse_bounded(xml, MAX_EVENT_NODES).map_err(|error| error.to_string())?;
        let event = parse::from_document(&document)?;
        if is_bookmark_event(&event)? {
            if document.root_element().children().any(|node| {
                node.is_text()
                    && node.text().is_some_and(|text| {
                        !text.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n'))
                    })
            }) {
                return Err("unexpected text in subscription bookmark event".into());
            }
            continue;
        }
        decoded.push(event);
        if include_event_original {
            originals.push(xml.as_str());
        }
    }
    let subscription_id = context.subscription_id.to_string();
    let mapping = map::MappingContext {
        scope_name: "windows_event_forwarding",
        extra_attributes: &[
            ("source.principal", AttrValue::Str(context.source.as_str())),
            ("source.address", AttrValue::Str(context.peer)),
            (
                "wef.subscription.name",
                AttrValue::Str(context.subscription_name),
            ),
            ("wef.subscription.uuid", AttrValue::Str(&subscription_id)),
            ("wef.subscription.version", AttrValue::Str(context.version)),
        ],
        event_originals: include_event_original.then_some(originals.as_slice()),
    };
    map::encode(&decoded, &mapping, observed_time_unix_nano)
}

/// Event ID 111 alone must never suppress an ordinary event.
fn is_bookmark_event(event: &WindowsEvent) -> Result<bool, String> {
    if event.system.event_id != 111
        || event.system.provider_name.as_deref() != Some("Microsoft-Windows-EventForwarder")
        || event
            .system
            .channel
            .as_deref()
            .is_some_and(|channel| !channel.is_empty())
        || event.rendering.is_some()
    {
        return Ok(false);
    }
    let Some(Payload::Other(marker)) = &event.payload else {
        return Ok(false);
    };
    if marker.name != "SubscriptionBookmarkEvent" || marker.namespace.as_deref() != Some(EVENT_NS) {
        return Ok(false);
    }
    let mut subscription_id_seen = false;
    for content in &marker.content {
        match content {
            Content::Text(text) if text.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n')) => {}
            Content::Element(child)
                if child.name == "SubscriptionId"
                    && child.namespace.as_deref() == Some(EVENT_NS)
                    && !subscription_id_seen
                    && child
                        .content
                        .iter()
                        .all(|item| matches!(item, Content::Text(_))) =>
            {
                subscription_id_seen = true;
            }
            _ => return Err("invalid subscription bookmark event".into()),
        }
    }
    if !subscription_id_seen {
        return Err("missing bookmark SubscriptionId".into());
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::super::{
        config::AuthConfig,
        wsman::{Envelope, Route, messages::Action},
    };
    use super::*;
    use arrow::{
        array::{
            Array, BinaryArray, Int32Array, StringArray, StructArray, TimestampNanosecondArray,
            UInt16Array,
        },
        compute::cast,
        datatypes::DataType,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

    fn source() -> SourceIdentity {
        let certified =
            rcgen::generate_simple_self_signed(vec!["host.example.com".into()]).unwrap();
        SourceIdentity::from_verified_certificate(
            certified.cert.der(),
            &AuthConfig {
                allowed_sources: vec!["host.example.com".into()],
            },
        )
        .unwrap()
    }

    fn context(source: &SourceIdentity) -> MappingContext<'_> {
        MappingContext {
            source,
            peer: "127.0.0.1:1234",
            subscription_name: "audit",
            subscription_id: Uuid::from_u128(42),
            version: "opaque-old-version",
        }
    }

    fn event(payload: &str) -> String {
        format!(
            r#"<Event xmlns="{EVENT_NS}"><System><Provider Name="Example"/><EventID>111</EventID><Level>2</Level><TimeCreated SystemTime="2026-09-22T19:28:11Z"/><Computer>untrusted-claim</Computer></System>{payload}</Event>"#
        )
    }

    fn marker() -> String {
        event("<SubscriptionBookmarkEvent><SubscriptionId/></SubscriptionBookmarkEvent>")
            .replace("Example", "Microsoft-Windows-EventForwarder")
    }

    fn strings(records: &OtapArrowRecords, key: &str) -> Vec<(u16, String)> {
        let attrs = records.get(ArrowPayloadType::LogAttrs).unwrap();
        let keys = cast(attrs.column_by_name("key").unwrap(), &DataType::Utf8).unwrap();
        let values = cast(attrs.column_by_name("str").unwrap(), &DataType::Utf8).unwrap();
        let keys = keys.as_any().downcast_ref::<StringArray>().unwrap();
        let values = values.as_any().downcast_ref::<StringArray>().unwrap();
        let ids = attrs
            .column_by_name("parent_id")
            .unwrap()
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap();
        (0..attrs.num_rows())
            .filter(|&i| keys.value(i) == key)
            .map(|i| (ids.value(i), values.value(i).to_owned()))
            .collect()
    }

    /// Scenario: SOAP delivers rendered and fallback events separated by a synthetic marker.
    /// Guarantees: shared mapping, source identity, order, original XML, and one observation time survive atomic conversion.
    #[test]
    fn maps_extracted_delivery_with_trusted_context() {
        let source = source();
        let context = context(&source);
        let first = event(
            "<RenderingInfo><Message>message &amp; text</Message><Level>localized</Level></RenderingInfo>",
        );
        let second = event("<EventData><Data Name='code'>value</Data></EventData>");
        let wrappers: String = [&first, &marker(), &second].into_iter()
            .map(|xml| format!(r#"<w:Event Action="http://schemas.dmtf.org/wbem/wsman/1/wsman/Event"><![CDATA[{xml}]]></w:Event>"#)).collect();
        let xml = format!(
            r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:a="http://schemas.xmlsoap.org/ws/2004/08/addressing" xmlns:w="http://schemas.dmtf.org/wbem/wsman/1/wsman.xsd" xmlns:e="http://schemas.xmlsoap.org/ws/2004/08/eventing"><s:Header><a:Action>{}</a:Action><a:MessageID>request</a:MessageID><a:To>https://collector/delivery</a:To><e:Identifier>{}</e:Identifier><w:AckRequested/></s:Header><s:Body><w:Events>{wrappers}</w:Events></s:Body></s:Envelope>"#,
            Action::Events.uri(),
            context.subscription_id
        );
        let envelope = Envelope::parse(&xml, xml.len(), 100).unwrap();
        let events = envelope
            .request(
                &Route::Delivery {
                    version: context.version,
                },
                8192,
            )
            .unwrap()
            .event_batch(&Limits::default())
            .unwrap();
        for include_original in [false, true] {
            let records =
                encode_events(&events, &context, &Limits::default(), include_original, 123)
                    .unwrap()
                    .unwrap();
            let logs = records.get(ArrowPayloadType::Logs).unwrap();
            assert_eq!(logs.num_rows(), 2);
            let observed = logs
                .column_by_name("observed_time_unix_nano")
                .unwrap()
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
                .unwrap();
            assert_eq!(observed.values().as_ref(), &[123, 123]);
            let timestamps = logs
                .column_by_name("time_unix_nano")
                .unwrap()
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
                .unwrap();
            let expected_time = chrono::DateTime::parse_from_rfc3339("2026-09-22T19:28:11Z")
                .unwrap()
                .timestamp_nanos_opt()
                .unwrap();
            assert_eq!(
                timestamps.values().as_ref(),
                &[expected_time, expected_time]
            );
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
            let body = logs
                .column_by_name("body")
                .unwrap()
                .as_any()
                .downcast_ref::<StructArray>()
                .unwrap();
            let text = cast(body.column_by_name("str").unwrap(), &DataType::Utf8).unwrap();
            let text = text.as_any().downcast_ref::<StringArray>().unwrap();
            assert_eq!(text.value(0), "message & text");
            assert!(text.is_null(1));
            let serialized = cast(body.column_by_name("ser").unwrap(), &DataType::Binary).unwrap();
            let fallback: serde_json::Value = serde_cbor::from_slice(
                serialized
                    .as_any()
                    .downcast_ref::<BinaryArray>()
                    .unwrap()
                    .value(1),
            )
            .unwrap();
            assert_eq!(fallback["event_data"]["entries"][0]["value"], "value");
            for (key, value) in [
                ("source.principal", source.as_str()),
                ("source.address", context.peer),
                ("wef.subscription.name", context.subscription_name),
                (
                    "wef.subscription.uuid",
                    &context.subscription_id.to_string(),
                ),
                ("wef.subscription.version", context.version),
                ("windows.eventlog.computer", "untrusted-claim"),
            ] {
                assert_eq!(
                    strings(&records, key),
                    vec![(0, value.into()), (1, value.into())]
                );
            }
            assert_eq!(
                strings(&records, "event.original"),
                if include_original {
                    vec![(0, first.clone()), (1, second.clone())]
                } else {
                    vec![]
                }
            );
        }
    }

    /// Scenario: marker-only batches, lookalike ordinary events, and malformed markers arrive.
    /// Guarantees: only validated marker shapes are omitted; malformed markers reject the whole batch.
    #[test]
    fn filters_only_validated_markers() {
        let source = source();
        let context = context(&source);
        assert!(
            encode_events(&[marker()], &context, &Limits::default(), true, 0)
                .unwrap()
                .is_none()
        );
        for ordinary in [
            event(""),
            marker().replace("Microsoft-Windows-EventForwarder", "Other"),
            marker().replace("<EventID>111", "<EventID>112"),
            marker().replace("</System>", "<Channel>Application</Channel></System>"),
            marker().replace("SubscriptionBookmarkEvent", "OtherPayload"),
            marker().replace(
                "<SubscriptionBookmarkEvent>",
                "<SubscriptionBookmarkEvent xmlns='urn:other'>",
            ),
        ] {
            assert!(
                encode_events(&[ordinary], &context, &Limits::default(), false, 0)
                    .unwrap()
                    .is_some()
            );
        }
        for content in [
            "",
            "<Unexpected/>",
            "text<SubscriptionId/>",
            "<SubscriptionId/><SubscriptionId/>",
            "<SubscriptionId><Nested/></SubscriptionId>",
        ] {
            let invalid = marker().replace("<SubscriptionId/>", content);
            assert!(
                encode_events(&[event(""), invalid], &context, &Limits::default(), true, 0)
                    .is_err()
            );
        }
    }

    /// Scenario: a valid prefix is followed by malformed XML or invalid Windows metadata.
    /// Guarantees: conversion returns an error, never a partial OTAP batch or a marker-only success.
    #[test]
    fn rejects_whole_batch_on_invalid_event() {
        let source = source();
        let context = context(&source);
        for invalid in [
            "<Event/>".into(),
            "<!DOCTYPE Event><Event/>".into(),
            event("").replace("<EventID>111", "<EventID>bad"),
            event("").replace("<Level>2", "<Level>256"),
            event("").replace("2026-09-22T19:28:11Z", "not-a-time"),
            marker().replace("<EventID>111", "<EventID>bad"),
            marker().replace("</System>", "</System>unexpected text"),
        ] {
            assert!(
                encode_events(
                    &[event(""), invalid],
                    &context,
                    &Limits::default(),
                    false,
                    0
                )
                .is_err()
            );
        }
    }

    /// Scenario: batches reach byte/node bounds or violate count and observation-time requirements.
    /// Guarantees: exact byte/node limits succeed; one-unit overflow and invalid batch metadata fail explicitly.
    #[test]
    fn bounds_conversion_inputs() {
        let source = source();
        let context = context(&source);
        let events = vec![event(""), event("")];
        let mut limits = Limits {
            max_event_bytes: events[0].len(),
            max_decompressed_bytes: events.iter().map(String::len).sum(),
            ..Limits::default()
        };
        assert!(
            encode_events(&events, &context, &limits, false, 0)
                .unwrap()
                .is_some()
        );
        limits.max_event_bytes -= 1;
        assert!(
            encode_events(&events, &context, &limits, false, 0)
                .unwrap_err()
                .contains("byte limits")
        );
        limits.max_event_bytes += 1;
        limits.max_decompressed_bytes -= 1;
        assert!(
            encode_events(&events, &context, &limits, false, 0)
                .unwrap_err()
                .contains("byte limits")
        );
        for events in [vec![], vec![String::new(); MAX_EVENTS + 1]] {
            assert!(
                encode_events(&events, &context, &Limits::default(), false, 0)
                    .unwrap_err()
                    .contains("batch size")
            );
        }
        assert!(
            encode_events(&[event("")], &context, &Limits::default(), false, -1)
                .unwrap_err()
                .contains("observation")
        );
        let base = event("");
        let base_nodes = Document::parse(&base).unwrap().descendants().count();
        let padded = event(&"<!--x-->".repeat(MAX_EVENT_NODES - base_nodes));
        assert!(
            encode_events(
                std::slice::from_ref(&padded),
                &context,
                &Limits::default(),
                false,
                0
            )
            .unwrap()
            .is_some()
        );
        let oversized = padded.replace("</Event>", "<!--x--></Event>");
        assert!(encode_events(&[oversized], &context, &Limits::default(), false, 0).is_err());
    }
}
