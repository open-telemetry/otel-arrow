// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Build a [`WindowsEvent`] from a bounded XML document.
//!
//! This resolves the `Event` root in the Windows event namespace, parses numeric
//! `System` identifiers, and retains `EventData` and `UserData` payloads. It does
//! not recognize synthetic markers, derive severity, or map attributes; those are
//! receiver concerns. Diagnostics are receiver conversion failures, not errors
//! reported by the event itself.

use super::super::xml::{Document, Node};
use super::model::{
    Attribute, Content, DataEntry, Element, EventData, RenderingInfo, System, WindowsEvent,
};

const EVENT_NS: &str = "http://schemas.microsoft.com/win/2004/08/events/event";
const MAX_USER_DATA_DEPTH: usize = 32;

/// Build a [`WindowsEvent`] from a parsed Event document.
///
/// # Errors
///
/// Returns a diagnostic string when the root is not an `Event` in the Windows
/// event namespace, required `System` fields are missing, numeric fields or the
/// timestamp are malformed, a scalar field contains nested elements, or `UserData`
/// nests beyond depth 32.
pub fn from_document(document: &Document<'_>) -> Result<WindowsEvent, String> {
    let root = document.root_element();
    if !root.has_tag_name((EVENT_NS, "Event")) {
        return Err("expected Windows Event root".into());
    }
    let system = parse_system(child(root, "System")?.ok_or("missing System")?)?;
    let rendering = child(root, "RenderingInfo")?
        .map(parse_rendering)
        .transpose()?;
    let event_data = parse_event_data(root)?;
    let user_data = child(root, "UserData")?
        .map(|node| parse_element(node, 0))
        .transpose()?;
    Ok(WindowsEvent {
        system,
        rendering,
        event_data,
        user_data,
    })
}

/// Parse the `System` section, typing numeric identifiers and the timestamp.
fn parse_system(system: Node<'_, '_>) -> Result<System, String> {
    let event_id = field(system, "EventID")?
        .ok_or("missing EventID")?
        .parse::<u32>()
        .map_err(|_| "invalid EventID")?;
    let time = child(system, "TimeCreated")?
        .and_then(|node| node.attribute("SystemTime").map(str::to_owned))
        .ok_or("missing SystemTime")?;
    let time_created_unix_nano = chrono::DateTime::parse_from_rfc3339(&time)
        .map_err(|_| "invalid SystemTime")?
        .timestamp_nanos_opt()
        .filter(|value| *value >= 0)
        .ok_or("SystemTime outside supported range")?;
    let level = field(system, "Level")?
        .map(|value| value.parse::<u8>().map_err(|_| "invalid Level"))
        .transpose()?
        .unwrap_or_default();
    let (provider_name, provider_guid) = match child(system, "Provider")? {
        Some(provider) => (
            provider.attribute("Name").map(str::to_owned),
            provider.attribute("Guid").map(str::to_owned),
        ),
        None => (None, None),
    };
    let user_sid = child(system, "Security")?
        .and_then(|security| security.attribute("UserID").map(str::to_owned));
    Ok(System {
        provider_name,
        provider_guid,
        event_id,
        level,
        time_created_unix_nano,
        channel: field(system, "Channel")?,
        record_id: parse_optional(system, "EventRecordID")?,
        task: parse_optional(system, "Task")?,
        opcode: parse_optional(system, "Opcode")?,
        keywords: field(system, "Keywords")?,
        computer: field(system, "Computer")?,
        user_sid,
    })
}

/// Parse optional `RenderingInfo`, distinguishing an empty message from absence.
fn parse_rendering(node: Node<'_, '_>) -> Result<RenderingInfo, String> {
    Ok(RenderingInfo {
        message: field(node, "Message")?,
        level: field(node, "Level")?,
    })
}

/// Collect `EventData/Data` entries in source order, naming unnamed ones `paramN`.
/// Collect the `EventData` payload, retaining `Data`, `ComplexData`, and `Binary`.
///
/// Unnamed `Data` entries are named `paramN` by their one-based element position.
fn parse_event_data(root: Node<'_, '_>) -> Result<EventData, String> {
    let Some(data) = child(root, "EventData")? else {
        return Ok(EventData::default());
    };
    let mut event_data = EventData::default();
    for (position, item) in data.children().filter(Node::is_element).enumerate() {
        if item.has_tag_name((EVENT_NS, "Data")) {
            let name = item
                .attribute("Name")
                .map(str::to_owned)
                .unwrap_or_else(|| format!("param{}", position + 1));
            event_data.entries.push(DataEntry {
                name,
                value: text(item)?,
            });
        } else if item.has_tag_name((EVENT_NS, "ComplexData")) {
            event_data.complex.push(parse_element(item, 0)?);
        } else if item.has_tag_name((EVENT_NS, "Binary")) {
            event_data.binary = Some(text(item)?);
        }
    }
    Ok(event_data)
}

/// Retain element/attribute names, namespaces, and ordered mixed content.
///
/// Comments and processing instructions are omitted. The depth limit bounds
/// recursion over untrusted documents.
fn parse_element(node: Node<'_, '_>, depth: usize) -> Result<Element, String> {
    if depth > MAX_USER_DATA_DEPTH {
        return Err("UserData exceeds maximum nesting depth of 32".into());
    }
    let attributes = node
        .attributes()
        .map(|attribute| Attribute {
            name: attribute.name().to_owned(),
            namespace: attribute.namespace().map(str::to_owned),
            value: attribute.value().to_owned(),
        })
        .collect();
    let mut content = Vec::new();
    for child in node.children() {
        if child.is_element() {
            content.push(Content::Element(parse_element(child, depth + 1)?));
        } else if child.is_text()
            && let Some(text) = child.text()
        {
            content.push(Content::Text(text.to_owned()));
        }
    }
    Ok(Element {
        name: node.tag_name().name().to_owned(),
        namespace: node.tag_name().namespace().map(str::to_owned),
        attributes,
        content,
    })
}

/// Parse a unique optional scalar field into a typed value.
fn parse_optional<T: std::str::FromStr>(
    node: Node<'_, '_>,
    name: &str,
) -> Result<Option<T>, String> {
    match field(node, name)? {
        Some(value) => Ok(Some(
            value.parse::<T>().map_err(|_| format!("invalid {name}"))?,
        )),
        None => Ok(None),
    }
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

    /// Scenario: a full event carries System metadata, rendering, EventData, and nested UserData.
    /// Guarantees: numeric System fields are typed, repeated and unnamed EventData entries are
    /// retained in order, and the UserData tree preserves names, namespaces, and text.
    #[test]
    fn builds_full_event() {
        let xml = concat!(
            "<Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'>",
            "<System>",
            "<Provider Name='Svc' Guid='{abc}'/>",
            "<EventID>42</EventID><Level>3</Level>",
            "<TimeCreated SystemTime='2026-09-22T19:28:11Z'/>",
            "<EventRecordID>99</EventRecordID><Task>7</Task><Opcode>1</Opcode>",
            "<Keywords>0x8000000000000000</Keywords><Channel>App</Channel>",
            "<Computer>host</Computer><Security UserID='S-1-5-18'/>",
            "</System>",
            "<EventData><Data Name='a'>one</Data><Data>two</Data><Data Name='a'>three</Data></EventData>",
            "<UserData><Payload xmlns='urn:p'>text<Inner k='v'/></Payload></UserData>",
            "<RenderingInfo><Message>rendered</Message><Level>Warning</Level></RenderingInfo>",
            "</Event>",
        );
        let document = Document::parse(xml).unwrap();
        let event = from_document(&document).unwrap();
        assert_eq!(event.system.provider_name.as_deref(), Some("Svc"));
        assert_eq!(event.system.event_id, 42);
        assert_eq!(event.system.level, 3);
        assert_eq!(event.system.record_id, Some(99));
        assert_eq!(event.system.task, Some(7));
        assert_eq!(event.system.opcode, Some(1));
        assert_eq!(event.system.keywords.as_deref(), Some("0x8000000000000000"));
        assert_eq!(event.system.channel.as_deref(), Some("App"));
        assert_eq!(event.system.user_sid.as_deref(), Some("S-1-5-18"));
        assert_eq!(
            event.system.time_created_unix_nano,
            1_790_105_291_000_000_000
        );
        let names: Vec<_> = event
            .event_data
            .entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect();
        let values: Vec<_> = event
            .event_data
            .entries
            .iter()
            .map(|entry| entry.value.as_str())
            .collect();
        assert_eq!(names, ["a", "param2", "a"]);
        assert_eq!(values, ["one", "two", "three"]);
        let user_data = event.user_data.unwrap();
        assert_eq!(user_data.name, "UserData");
        assert_eq!(user_data.content.len(), 1);
        let Content::Element(payload) = &user_data.content[0] else {
            panic!("expected Payload element");
        };
        assert_eq!(payload.name, "Payload");
        assert_eq!(payload.namespace.as_deref(), Some("urn:p"));
        assert_eq!(payload.content.len(), 2);
        assert_eq!(payload.content[0], Content::Text("text".into()));
        let Content::Element(inner) = &payload.content[1] else {
            panic!("expected nested element");
        };
        assert_eq!(inner.name, "Inner");
        assert_eq!(inner.attributes[0].name, "k");
        let rendering = event.rendering.unwrap();
        assert_eq!(rendering.message.as_deref(), Some("rendered"));
        assert_eq!(rendering.level.as_deref(), Some("Warning"));
    }

    /// Scenario: an absent message with no payload yields an empty structured representation.
    /// Guarantees: absence of RenderingInfo and payload sections is distinguished from empty content.
    #[test]
    fn omits_absent_sections() {
        let xml = concat!(
            "<Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'>",
            "<System><EventID>1</EventID><TimeCreated SystemTime='2026-09-22T19:28:11Z'/></System>",
            "</Event>",
        );
        let event = from_document(&Document::parse(xml).unwrap()).unwrap();
        assert_eq!(event.system.event_id, 1);
        assert_eq!(event.system.level, 0);
        assert!(event.system.channel.is_none());
        assert!(event.rendering.is_none());
        assert!(event.event_data.entries.is_empty());
        assert!(event.event_data.complex.is_empty());
        assert!(event.event_data.binary.is_none());
        assert!(event.user_data.is_none());
    }

    /// Scenario: EventData carries ComplexData and a Binary payload alongside Data entries.
    /// Guarantees: every EventData variant is retained rather than silently dropped.
    #[test]
    fn retains_complex_and_binary_event_data() {
        let xml = concat!(
            "<Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'>",
            "<System><EventID>1</EventID><TimeCreated SystemTime='2026-09-22T19:28:11Z'/></System>",
            "<EventData>",
            "<Data Name='a'>one</Data>",
            "<ComplexData><Field k='v'>inner</Field></ComplexData>",
            "<Binary>0102FF</Binary>",
            "</EventData>",
            "</Event>",
        );
        let event = from_document(&Document::parse(xml).unwrap()).unwrap();
        assert_eq!(event.event_data.entries.len(), 1);
        assert_eq!(event.event_data.entries[0].name, "a");
        assert_eq!(event.event_data.binary.as_deref(), Some("0102FF"));
        assert_eq!(event.event_data.complex.len(), 1);
        let complex = &event.event_data.complex[0];
        assert_eq!(complex.name, "ComplexData");
        let Content::Element(field) = &complex.content[0] else {
            panic!("expected nested Field element");
        };
        assert_eq!(field.name, "Field");
        assert_eq!(field.attributes[0].name, "k");
        assert_eq!(field.content[0], Content::Text("inner".into()));
    }

    /// Scenario: malformed documents miss required fields, carry bad numbers, or nest too deeply.
    /// Guarantees: conversion fails closed rather than producing a partial event.
    #[test]
    fn rejects_malformed_events() {
        let ns = "http://schemas.microsoft.com/win/2004/08/events/event";
        let deep_user_data = format!(
            "<Event xmlns='{ns}'><System><EventID>1</EventID><TimeCreated SystemTime='2026-09-22T19:28:11Z'/></System><UserData>{}{}</UserData></Event>",
            "<n>".repeat(40),
            "</n>".repeat(40),
        );
        for xml in [
            format!("<NotEvent xmlns='{ns}'/>"),
            format!("<Event xmlns='{ns}'><RenderingInfo/></Event>"),
            format!(
                "<Event xmlns='{ns}'><System><EventID>x</EventID><TimeCreated SystemTime='2026-09-22T19:28:11Z'/></System></Event>"
            ),
            format!("<Event xmlns='{ns}'><System><EventID>1</EventID></System></Event>"),
            format!(
                "<Event xmlns='{ns}'><System><EventID>1</EventID><TimeCreated SystemTime='not-a-time'/></System></Event>"
            ),
            format!(
                "<Event xmlns='{ns}'><System><EventID>1</EventID><TimeCreated SystemTime='2026-09-22T19:28:11Z'/><EventRecordID>x</EventRecordID></System></Event>"
            ),
            deep_user_data,
        ] {
            assert!(
                from_document(&Document::parse(&xml).unwrap()).is_err(),
                "accepted {xml}"
            );
        }
    }
}
